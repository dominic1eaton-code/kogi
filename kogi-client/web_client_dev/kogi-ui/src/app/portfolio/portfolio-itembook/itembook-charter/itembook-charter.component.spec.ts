import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ItembookCharterComponent } from './itembook-charter.component';

describe('ItembookCharterComponent', () => {
  let component: ItembookCharterComponent;
  let fixture: ComponentFixture<ItembookCharterComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ItembookCharterComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(ItembookCharterComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
