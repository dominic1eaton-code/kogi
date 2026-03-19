import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ItembookScheduleComponent } from './itembook-schedule.component';

describe('ItembookScheduleComponent', () => {
  let component: ItembookScheduleComponent;
  let fixture: ComponentFixture<ItembookScheduleComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ItembookScheduleComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(ItembookScheduleComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
