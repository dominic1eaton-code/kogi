import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ItembookLibraryComponent } from './itembook-library.component';

describe('ItembookLibraryComponent', () => {
  let component: ItembookLibraryComponent;
  let fixture: ComponentFixture<ItembookLibraryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ItembookLibraryComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(ItembookLibraryComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
